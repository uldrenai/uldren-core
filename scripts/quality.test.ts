import { type CommandRunner, CoverageGate, QualityTools } from "./quality.ts";

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
  const gate = new CoverageGate({ domain: { lines: 90, branches: 85 } }, []);
  gate.check([file(90, 85)]);
  rejects(() => gate.check([file(89, 100)]), "domain lines");
  rejects(() => gate.check([file(100, 84)]), "domain branches");
  rejects(() => gate.check([]), "no coverage data");
  rejects(() => gate.check([file(101, 100)]), "invalid lines");
  const noBranches = file(100, 0);
  noBranches.summary.branches.count = 0;
  rejects(() => gate.check([noBranches]), "no branch coverage data");
  rejects(
    () => new CoverageGate({ domain: { lines: NaN, branches: 85 } }, []).check([file(100, 100)]),
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
