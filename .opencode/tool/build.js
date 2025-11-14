import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "Build the Rust project",
  args: {
    all: tool.schema.boolean().default(true).describe("Build all packages"),
    release: tool.schema.boolean().default(false).describe("Build in release mode"),
    target: tool.schema.string().optional().describe("Specific target to build"),
  },
  async execute(args) {
    const { all, release, target } = args

    let command = "cargo build"

    if (all) {
      command += " --all"
    }

    if (release) {
      command += " --release"
    }

    if (target) {
      command += ` --target ${target}`
    }

    console.log(`Building project: ${command}`)

    try {
      const result = await Bun.$(command)
      return `✅ Build successful!\n${result.stdout || "No output"}`
    } catch (error) {
      return `❌ Build failed!\n${error.stderr || error.message}`
    }
  },
})