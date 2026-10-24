const source = Deno.cwd();
const config = JSON.parse(await Deno.readTextFile("tooling/quality.json"));
const toolsPrefix = Deno.env.get("ULDREN_QUALITY_TOOLS") ?? `${source}/target/quality-tools`;

class EnforcementSmoke {
  constructor(readonly root: string) {}

  async copy(directory: string, destination: string): Promise<void> {
    await Deno.mkdir(destination, { recursive: true });
    for await (const entry of Deno.readDir(directory)) {
      if ([".git", "target", "corpus", "artifacts", "coverage"].includes(entry.name)) continue;
      const from = `${directory}/${entry.name}`;
      const to = `${destination}/${entry.name}`;
      if (entry.isDirectory) await this.copy(from, to);
      else if (entry.isFile) await Deno.copyFile(from, to);
      else throw new Error(`unsupported fixture entry: ${from}`);
    }
  }

  async command(directory: string, args: string[], expected?: RegExp): Promise<void> {
    const result = await new Deno.Command(args[0], {
      args: args.slice(1),
      cwd: directory,
      env: {
        ULDREN_QUALITY_TOOLS: toolsPrefix,
        CARGO_TARGET_DIR: `${this.root}/build`,
      },
      stdout: "piped",
      stderr: "piped",
    }).output();
    const text = new TextDecoder().decode(result.stdout) + new TextDecoder().decode(result.stderr);
    await Deno.writeTextFile(`${this.root}/${this.sequence++}.log`, text);
    if (expected ? result.success || !expected.test(text) : !result.success) {
      console.error(text);
      throw new Error(`unexpected result from ${args.join(" ")}: ${result.code}`);
    }
  }

  sequence = 0;

  async mutate(path: string, from: string, to: string): Promise<void> {
    const contents = await Deno.readTextFile(path);
    if (!contents.includes(from)) throw new Error(`mutation anchor missing: ${path}`);
    await Deno.writeTextFile(path, contents.replace(from, to));
  }

  async run(): Promise<void> {
    const checkout = `${this.root}/checkout`;
    await this.copy(source, checkout);
    const identifier = `${checkout}/crates/domain/src/identifier.rs`;
    const original = await Deno.readTextFile(identifier);

    await this.command(checkout, ["just", "fmt-check"]);
    await Deno.writeTextFile(
      identifier,
      original.replace("pub fn as_str(&self)", "pub fn as_str( &self )"),
    );
    await this.command(checkout, ["just", "fmt-check"], /Diff in .*identifier\.rs/);
    await Deno.writeTextFile(identifier, original);
    console.log("PASS: formatter rejects malformed source");

    await Deno.writeTextFile(
      identifier,
      `${original}\npub fn quality_lint_fixture() -> bool { true == true }\n`,
    );
    await this.command(checkout, ["just", "lint"], /equal expressions|clippy::eq_op/);
    await Deno.writeTextFile(identifier, original);
    console.log("PASS: Clippy rejects an intentional lint");

    const policy = `${checkout}/deny.toml`;
    await this.mutate(policy, '    "Apache-2.0",\n', "");
    await this.mutate(policy, '    "MIT",\n', "");
    const deny = `${toolsPrefix}/bin/cargo-deny`;
    await this.command(
      checkout,
      [deny, "--locked", "check", "licenses"],
      /license.*not.*allowed|unlicensed/,
    );
    await Deno.copyFile(`${source}/deny.toml`, policy);
    console.log("PASS: dependency policy rejects a disallowed license");

    const manifest = `${checkout}/crates/contracts/Cargo.toml`;
    const manifestText = await Deno.readTextFile(manifest);
    await Deno.writeTextFile(manifest, `${manifestText}\n[dependencies]\nzeroize = "=1.9.0"\n`);
    await this.command(checkout, [`${toolsPrefix}/bin/cargo-machete`, "."], /unused dependencies/);
    await Deno.writeTextFile(manifest, manifestText);
    console.log("PASS: unused dependency detection rejects an injected dependency");

    const configPath = `${checkout}/tooling/quality.json`;
    const raised = structuredClone(config);
    for (const floor of Object.values(raised.coverage) as { lines: number }[]) floor.lines = 100;
    await Deno.writeTextFile(configPath, JSON.stringify(raised));
    await this.command(checkout, ["just", "coverage"], /coverage floor failed/);
    await Deno.copyFile(`${source}/tooling/quality.json`, configPath);
    console.log("PASS: coverage rejects an unmet component floor");

    const baseline = `${this.root}/baseline`;
    await this.copy(checkout, baseline);
    await this.mutate(identifier, "pub fn kind(&self)", "pub(crate) fn kind(&self)");
    await this.command(checkout, [
      `${toolsPrefix}/bin/cargo-semver-checks`,
      "check-release",
      "--package",
      "uldren-core-domain",
      "--baseline-root",
      baseline,
      "--default-features",
      "--release-type",
      "patch",
    ], /inherent_method_missing|major.*required|semver requires/);
    await Deno.writeTextFile(identifier, original);
    console.log("PASS: semver detects a removed public method");

    const fuzz = `${checkout}/fuzz/fuzz_targets/entity_id.rs`;
    await this.mutate(
      fuzz,
      "let result = EntityId::parse(input);",
      'panic!("quality fuzz failure fixture");\n    let result = EntityId::parse(input);',
    );
    await this.command(checkout, ["just", "fuzz-smoke"], /quality fuzz failure fixture/);
    console.log("PASS: fuzzing propagates a target crash");
  }
}

const directory = await Deno.makeTempDir({ prefix: "uldren-quality-smoke-" });
console.log(`Enforcement evidence: ${directory}`);
await new EnforcementSmoke(directory).run();
