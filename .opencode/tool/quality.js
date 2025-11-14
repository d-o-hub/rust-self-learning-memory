import { tool } from "@opencode-ai/plugin"

export const runTests = tool({
  description: "Run tests for the Rust project",
  args: {
    all: tool.schema.boolean().default(true).describe("Run all tests"),
    package: tool.schema.string().optional().describe("Specific package to test"),
    nocapture: tool.schema.boolean().default(false).describe("Show test output"),
  },
  async execute(args) {
    const { all, package, nocapture } = args

    let command = "cargo test"

    if (!all && package) {
      command += ` -p ${package}`
    } else if (all) {
      command += " --all"
    }

    if (nocapture) {
      command += " -- --nocapture"
    }

    console.log(`Running: ${command}`)

    try {
      const result = await Bun.$(command)
      return `✅ Tests passed!\n${result.stdout || result.stderr || "No output"}`
    } catch (error) {
      return `❌ Tests failed!\n${error.stderr || error.message}`
    }
  },
})

export const checkQuality = tool({
  description: "Run quality checks (format, clippy, audit)",
  args: {},
  async execute() {
    const results = []

    try {
      console.log("Checking format...")
      await Bun.$`cargo fmt -- --check`
      results.push("✅ Format check passed")
    } catch {
      results.push("❌ Format check failed")
    }

    try {
      console.log("Running clippy...")
      await Bun.$`cargo clippy -- -D warnings`
      results.push("✅ Clippy check passed")
    } catch {
      results.push("❌ Clippy check failed")
    }

    try {
      console.log("Auditing dependencies...")
      await Bun.$`cargo audit`
      results.push("✅ Security audit passed")
    } catch {
      results.push("⚠️  Security audit found issues")
    }

    return results.join("\n")
  },
})