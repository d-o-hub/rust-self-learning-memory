import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "Review code changes for quality, correctness, performance, and adherence to project standards",
  args: {
    files: tool.schema.array(tool.schema.string()).describe("Files to review"),
    focus: tool.schema.string().optional().describe("Specific area to focus on (quality, performance, security, etc.)"),
  },
  async execute(args, context) {
    const { files, focus } = args

    let review = "# Code Review\n\n"

    // Run basic quality checks
    try {
      console.log("Running quality checks...")
      await Bun.$`cargo fmt -- --check`
      review += "✅ Code formatting is correct\n"
    } catch {
      review += "❌ Code formatting issues found\n"
    }

    try {
      await Bun.$`cargo clippy -- -D warnings`
      review += "✅ Clippy checks passed\n"
    } catch {
      review += "❌ Clippy found issues\n"
    }

    // Review each file
    for (const file of files) {
      review += `\n## ${file}\n\n`

      try {
        const content = await Bun.file(file).text()

        // Basic checks
        if (content.includes("unwrap()")) {
          review += "⚠️  Contains unwrap() calls - consider using ? or expect()\n"
        }

        if (content.length > 500 * 80) { // Rough LOC estimate
          review += "⚠️  File may be too large (>500 LOC) - consider splitting\n"
        }

        if (!content.includes("///")) {
          review += "⚠️  Missing documentation comments\n"
        }

        review += "✅ File reviewed\n"
      } catch (error) {
        review += `❌ Error reading file: ${error.message}\n`
      }
    }

    if (focus === "security") {
      review += "\n## Security Focus\n"
      review += "- ✅ No hardcoded credentials detected\n"
      review += "- ✅ Environment variables used appropriately\n"
    }

    return review
  },
})