import { type CommandRunner, CoverageArtifacts, CoverageGate, QualityTools } from "./quality.ts";

const config = JSON.parse(await Deno.readTextFile("tooling/quality.json"));

function rejects(operation: () => void, expected: string): void {
  try {
    operation();
  } catch (error) {
    if (error instanceof Error && error.message.includes(expected)) return;
    throw error;
  }
  throw new Error(`expected rejection: ${expected}`);
}

const file = (lines: number, branches: number) => ({
  filename: "/workspace/crates/domain/src/identifier.rs",
  summary: {
    lines: { count: 100, covered: lines },
    branches: { count: 100, covered: branches },
  },
});

Deno.test("coverage enforces exact component boundaries", () => {
  const gate = new CoverageGate({ domain: { lines: 90, branches: 85 } }, [], []);
  gate.check([file(90, 85)]);
  rejects(() => gate.check([file(89, 100)]), "domain lines");
  rejects(() => gate.check([file(100, 84)]), "domain branches");
  rejects(() => gate.check([]), "no coverage data");
  rejects(() => gate.check([file(101, 100)]), "invalid lines");
  const noBranches = file(100, 0);
  noBranches.summary.branches.count = 0;
  rejects(() => gate.check([noBranches]), "no branch coverage data");
  rejects(
    () =>
      new CoverageGate({ domain: { lines: NaN, branches: 85 } }, [], []).check([file(100, 100)]),
    "invalid coverage floor",
  );
  rejects(
    () => gate.check([{ ...file(100, 100), filename: "crates/new/src/lib.rs" }]),
    "missing coverage policy",
  );
  rejects(
    () => gate.check([{ ...file(100, 100), filename: "tests/fixture.rs" }]),
    "unexpected coverage source",
  );
});

Deno.test("branchless components are explicit without accepting missing instrumentation", () => {
  const gate = new CoverageGate(
    {
      domain: { lines: 90, branches: 85 },
      security: { lines: 90, branches: 85 },
    },
    [],
    ["security"],
  );
  const security = {
    filename: "crates/security/src/secret.rs",
    summary: { lines: { count: 10, covered: 10 }, branches: { count: 0, covered: 0 } },
  };
  gate.check([file(100, 100), security]);
  rejects(() => gate.check([file(100, 84), security]), "domain branches");
  const uncovered = structuredClone(security);
  uncovered.summary.lines.covered = 8;
  rejects(() => gate.check([file(100, 100), uncovered]), "security lines");
  rejects(() => gate.check([file(100, 100)]), "no coverage data: security");
  const noBranches = file(100, 0);
  noBranches.summary.branches.count = 0;
  rejects(() => gate.check([noBranches, security]), "no branch coverage data");
  rejects(
    () => new CoverageGate(gate.floors, [], []).check([file(100, 100), security]),
    "unexpected zero branch count: security",
  );
});

Deno.test("new branches require removing the expectation and meeting the retained floor", () => {
  const floors = {
    domain: { lines: 90, branches: 85 },
    security: { lines: 90, branches: 85 },
  };
  const security = { ...file(100, 84), filename: "crates/security/src/secret.rs" };
  const gate = new CoverageGate(floors, [], ["security"]);
  rejects(() => gate.check([file(100, 100), security]), "stale branchless expectation: security");
  security.summary.branches.covered = 100;
  rejects(() => gate.check([file(100, 100), security]), "stale branchless expectation: security");
  const enforced = new CoverageGate(floors, [], []);
  enforced.check([file(100, 100), security]);
  security.summary.branches.covered = 84;
  rejects(() => enforced.check([file(100, 100), security]), "security branches");
  const split = { ...file(100, 0), filename: "crates/security/src/another.rs" };
  split.summary.branches.count = 0;
  rejects(
    () => gate.check([file(100, 100), split, security]),
    "stale branchless expectation: security",
  );
});

Deno.test("zero branch floors do not bypass branchless expectations", () => {
  const floors = { domain: { lines: 80, branches: 0 } };
  const gate = new CoverageGate(floors, [], ["domain"]);
  rejects(() => gate.check([file(100, 100)]), "stale branchless expectation: domain");
  const noBranches = file(100, 0);
  noBranches.summary.branches.count = 0;
  gate.check([noBranches]);
  rejects(
    () => new CoverageGate(floors, [], []).check([noBranches]),
    "unexpected zero branch count: domain",
  );
});

Deno.test("branchless expectations reject unknown and duplicate components", () => {
  const floors = { domain: { lines: 90, branches: 85 } };
  rejects(() => new CoverageGate(floors, [], ["typo"]), "unknown branchless component: typo");
  rejects(
    () => new CoverageGate(floors, [], ["toString"]),
    "unknown branchless component: toString",
  );
  rejects(() => new CoverageGate(floors, ["uniffi"], ["uniffi"]), "unknown branchless component");
  rejects(
    () => new CoverageGate(floors, [], ["domain", "domain"]),
    "duplicate branchless expectation",
  );
});

Deno.test("coverage binary selection accepts only exact Cargo targets", () => {
  const artifacts = new CoverageArtifacts(["uldren-core-contracts", "adapter_tests"]);
  for (const name of ["uldren_core_contracts-a1b2", "adapter_tests-123.exe", "adapter_tests"]) {
    if (!artifacts.pattern.test(name)) throw new Error(`valid target rejected: ${name}`);
  }
  for (
    const name of [
      "libuldren_core_contracts.rlib",
      "adapter_tests-123.d",
      "unrelated-abc",
      "adapter_tests_extra",
    ]
  ) {
    if (artifacts.pattern.test(name)) throw new Error(`unrelated artifact accepted: ${name}`);
  }
  rejects(() => new CoverageArtifacts([]), "invalid Cargo coverage target");
  rejects(() => new CoverageArtifacts(["invalid.*"]), "invalid Cargo coverage target");
});

class RecordingRunner implements CommandRunner {
  readonly calls: string[][] = [];
  fail = false;
  version = "";

  run(args: string[]): Promise<string> {
    this.calls.push(args);
    if (args.includes("--version")) return Promise.resolve(`tool ${this.version}\n`);
    if (this.fail) return Promise.reject(new Error("tool execution failed"));
    return Promise.resolve("");
  }
}

Deno.test("dependency policy selects all manifests and uses the pinned CLI contract", async () => {
  const runner = new RecordingRunner();
  runner.version = config.tools["cargo-deny"];
  await new QualityTools(config, runner).run("deny");
  const checks = runner.calls.filter((args) => args.includes("check"));
  if (checks.length !== 3) throw new Error("dependency graph omitted");
  for (const args of checks) {
    if (args.indexOf("--config") > args.indexOf("check")) {
      throw new Error("config must be a global cargo-deny option");
    }
    if (!args.includes("--locked")) throw new Error("dependency graph is not locked");
  }
});

for (const [tool, version] of Object.entries(config.tools)) {
  Deno.test(`${tool} validates its pin and propagates command failure`, async () => {
    const runner = new RecordingRunner();
    const tools = new QualityTools(config, runner);
    runner.version = String(version);
    await tools.tool(tool, ["test-argument"]);
    if (!runner.calls.at(-1)?.includes("test-argument")) throw new Error("arguments lost");
    runner.fail = true;
    try {
      await tools.tool(tool, ["test-argument"]);
      throw new Error("failure was swallowed");
    } catch (error) {
      if (!(error instanceof Error) || error.message !== "tool execution failed") throw error;
    }
    runner.version = "0.0.0";
    const before = runner.calls.length;
    try {
      await tools.tool(tool, ["test-argument"]);
      throw new Error("wrong version accepted");
    } catch (error) {
      if (!(error instanceof Error) || !error.message.includes("wrong ")) throw error;
    }
    if (runner.calls.length !== before + 1) throw new Error("ran a wrong-version tool");
  });
}
