export interface CommandRunner {
  run(args: string[], capture?: boolean): Promise<string>;
}

export class ProcessRunner implements CommandRunner {
  constructor(readonly cwd = Deno.cwd()) {}

  async run(args: string[], capture = false): Promise<string> {
    console.error(`> ${args.join(" ")}`);
    const process = new Deno.Command(args[0], {
      args: args.slice(1),
      cwd: this.cwd,
      stdout: capture ? "piped" : "inherit",
      stderr: "inherit",
    });
    if (capture) {
      const result = await process.output();
      if (!result.success) throw new Error(`${args[0]} failed (${result.code})`);
      return new TextDecoder().decode(result.stdout);
    }
    const status = await process.spawn().status;
    if (!status.success) throw new Error(`${args[0]} failed (${status.code})`);
    return "";
  }
}

interface Floor {
  lines: number;
  branches: number;
}

interface Configuration {
  tools: Record<string, string>;
  nightly: string;
  coverage: Record<string, Floor>;
  nonExecutableCrates: string[];
}

interface Metric {
  count: number;
  covered: number;
}

interface CoverageFile {
  filename: string;
  summary: { lines: Metric; branches: Metric };
}

export class CoverageGate {
  constructor(readonly floors: Record<string, Floor>, readonly nonExecutable: string[]) {}

  check(files: CoverageFile[]): void {
    const totals = new Map<string, { lines: Metric; branches: Metric }>();
    for (const file of files) {
      const component = file.filename.replaceAll("\\", "/").match(/(?:^|\/)crates\/([^/]+)\/src\//)
        ?.[1];
      if (!component) throw new Error(`unexpected coverage source: ${file.filename}`);
      if (!(component in this.floors)) {
        if (this.nonExecutable.includes(component) && file.summary.lines.count === 0) continue;
        throw new Error(`missing coverage policy: ${component}`);
      }
      const total = totals.get(component) ?? {
        lines: { count: 0, covered: 0 },
        branches: { count: 0, covered: 0 },
      };
      for (const metric of ["lines", "branches"] as const) {
        const value = file.summary[metric];
        if (
          !value || !Number.isInteger(value.count) || !Number.isInteger(value.covered) ||
          value.count < 0 || value.covered < 0 || value.covered > value.count
        ) {
          throw new Error(`invalid ${metric} metric for ${component}`);
        }
        total[metric].count += value.count;
        total[metric].covered += value.covered;
      }
      totals.set(component, total);
    }
    for (const [component, floor] of Object.entries(this.floors)) {
      const total = totals.get(component);
      if (!total || total.lines.count === 0) throw new Error(`no coverage data: ${component}`);
      for (const metric of ["lines", "branches"] as const) {
        const value = total[metric];
        if (!Number.isFinite(floor[metric]) || floor[metric] < 0 || floor[metric] > 100) {
          throw new Error(`invalid coverage floor: ${component} ${metric}`);
        }
        if (metric === "branches" && floor.branches > 0 && value.count === 0) {
          throw new Error(`no branch coverage data: ${component}`);
        }
        const percent = value.count === 0 ? 100 : value.covered * 100 / value.count;
        console.log(`${component} ${metric}: ${percent.toFixed(2)}% (floor ${floor[metric]}%)`);
        if (percent < floor[metric]) {
          throw new Error(`coverage floor failed: ${component} ${metric}`);
        }
      }
    }
  }
}

export class QualityTools {
  readonly prefix: string;

  constructor(
    readonly config: Configuration,
    readonly runner: CommandRunner = new ProcessRunner(),
  ) {
    this.prefix = Deno.env.get("ULDREN_QUALITY_TOOLS") ?? `${Deno.cwd()}/target/quality-tools`;
  }

  async tool(name: string, args: string[]): Promise<void> {
    const executable = `${this.prefix}/bin/${name}${Deno.build.os === "windows" ? ".exe" : ""}`;
    const subcommand = name === "cargo-llvm-cov" ? ["llvm-cov"] : [];
    const output = await this.runner.run([executable, ...subcommand, "--version"], true);
    if (output.trim().split(/\s+/).at(-1) !== this.config.tools[name]) {
      throw new Error(`wrong ${name} version; run just tools-install`);
    }
    await this.runner.run([executable, ...subcommand, ...args]);
  }

  async install(): Promise<void> {
    for (const [name, version] of Object.entries(this.config.tools)) {
      await this.runner.run([
        "cargo",
        "install",
        "--locked",
        "--root",
        this.prefix,
        name,
        "--version",
        version,
      ]);
    }
    await this.runner.run([
      "rustup",
      "toolchain",
      "install",
      this.config.nightly,
      "--profile",
      "minimal",
      "--component",
      "llvm-tools-preview,rustfmt",
    ]);
    await this.runner.run(["cargo", "fetch", "--manifest-path", "fuzz/Cargo.toml", "--locked"]);
  }

  async coverage(): Promise<void> {
    await Deno.mkdir("target/quality", { recursive: true });
    const previous = Deno.env.get("RUSTUP_TOOLCHAIN");
    const previousTarget = Deno.env.get("CARGO_LLVM_COV_TARGET_DIR");
    Deno.env.set("RUSTUP_TOOLCHAIN", this.config.nightly);
    Deno.env.set("CARGO_LLVM_COV_TARGET_DIR", `${Deno.cwd()}/target/quality/coverage-build`);
    try {
      await this.tool("cargo-llvm-cov", ["clean", "--workspace"]);
      await this.tool("cargo-llvm-cov", [
        "clean",
        "--manifest-path",
        "tests/contract/production-adapters/Cargo.toml",
        "--workspace",
      ]);
      await this.tool("cargo-llvm-cov", [
        "--workspace",
        "--all-features",
        "--locked",
        "--branch",
        "--no-report",
      ]);
      await this.tool("cargo-llvm-cov", [
        "--manifest-path",
        "tests/contract/production-adapters/Cargo.toml",
        "--locked",
        "--branch",
        "--no-report",
      ]);
      await this.tool("cargo-llvm-cov", [
        "report",
        "--branch",
        "--all-features",
        "--json",
        "--output-path",
        "target/quality/coverage.json",
      ]);
    } finally {
      if (previous === undefined) Deno.env.delete("RUSTUP_TOOLCHAIN");
      else Deno.env.set("RUSTUP_TOOLCHAIN", previous);
      if (previousTarget === undefined) Deno.env.delete("CARGO_LLVM_COV_TARGET_DIR");
      else Deno.env.set("CARGO_LLVM_COV_TARGET_DIR", previousTarget);
    }
    const report = JSON.parse(await Deno.readTextFile("target/quality/coverage.json"));
    new CoverageGate(this.config.coverage, this.config.nonExecutableCrates).check(
      report.data.flatMap((entry: { files: CoverageFile[] }) => entry.files),
    );
  }

  async semver(baseline: string): Promise<void> {
    const revision =
      (await this.runner.run(["git", "rev-parse", "--verify", `${baseline}^{commit}`], true))
        .trim();
    const metadata = JSON.parse(
      await this.runner.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
        true,
      ),
    );
    for (const pkg of metadata.packages) {
      await this.tool("cargo-semver-checks", [
        "check-release",
        "--package",
        pkg.name,
        "--baseline-rev",
        revision,
        "--default-features",
        "--release-type",
        "patch",
      ]);
    }
  }

  async run(command: string, argument?: string): Promise<void> {
    switch (command) {
      case "install":
        return await this.install();
      case "coverage":
        return await this.coverage();
      case "deny":
        for (
          const manifest of [
            "Cargo.toml",
            "tests/contract/production-adapters/Cargo.toml",
            "fuzz/Cargo.toml",
          ]
        ) {
          await this.tool("cargo-deny", [
            "--manifest-path",
            manifest,
            "--locked",
            "--config",
            "deny.toml",
            "check",
          ]);
        }
        return;
      case "machete":
        return await this.tool("cargo-machete", ["."]);
      case "semver":
        return await this.semver(argument ?? "origin/main");
      case "smoke-if-changed": {
        const baseline = argument ?? "origin/main";
        const changed = await this.runner.run([
          "git",
          "diff",
          "--name-only",
          baseline,
          "HEAD",
          "--",
          "scripts/quality.ts",
          "scripts/quality.test.ts",
          "scripts/quality-smoke.ts",
          "tooling",
          "justfile",
          "Cargo.toml",
          "rust-toolchain.toml",
          "rustfmt.toml",
          "clippy.toml",
          "deny.toml",
          "fuzz",
        ], true);
        if (changed.trim()) {
          await this.runner.run(["just", "fuzz-smoke"]);
          await this.runner.run(["just", "tooling-smoke"]);
        } else console.log("quality tooling unchanged; enforcement smoke runs on schedule");
        return;
      }
      case "fuzz": {
        await this.runner.run([
          "cargo",
          "metadata",
          "--manifest-path",
          "fuzz/Cargo.toml",
          "--format-version",
          "1",
          "--locked",
          "--offline",
        ], true);
        const lock = await Deno.readTextFile("fuzz/Cargo.lock");
        const previousToolchain = Deno.env.get("RUSTUP_TOOLCHAIN");
        const previousOffline = Deno.env.get("CARGO_NET_OFFLINE");
        Deno.env.set("RUSTUP_TOOLCHAIN", this.config.nightly);
        Deno.env.set("CARGO_NET_OFFLINE", "true");
        let failure: unknown;
        try {
          await this.tool("cargo-fuzz", [
            "run",
            "entity_id",
            "--",
            "-runs=1000",
            "-max_len=256",
            "-timeout=2",
            "-seed=1",
          ]);
        } catch (error) {
          failure = error;
        } finally {
          if (previousToolchain === undefined) Deno.env.delete("RUSTUP_TOOLCHAIN");
          else Deno.env.set("RUSTUP_TOOLCHAIN", previousToolchain);
          if (previousOffline === undefined) Deno.env.delete("CARGO_NET_OFFLINE");
          else Deno.env.set("CARGO_NET_OFFLINE", previousOffline);
        }
        if (await Deno.readTextFile("fuzz/Cargo.lock") !== lock) {
          throw new Error("fuzz lockfile changed", { cause: failure });
        }
        if (failure) throw failure;
        return;
      }
      default:
        throw new Error(`unknown quality command: ${command}`);
    }
  }
}

if (import.meta.main) {
  const config = JSON.parse(await Deno.readTextFile("tooling/quality.json")) as Configuration;
  await new QualityTools(config).run(Deno.args[0], Deno.args[1]);
}
