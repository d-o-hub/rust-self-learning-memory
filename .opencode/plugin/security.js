export const SecurityPlugin = async ({ project, client, $, directory, worktree }) => {
  console.log("🔒 Security plugin initialized!")

  return {
    "tool.execute.before": async (input, output) => {
      // Prevent reading .env files
      if (input.tool === "read" && output.args.filePath?.includes(".env")) {
        throw new Error("Do not read .env files - use environment variables instead")
      }
    },

    event: async ({ event }) => {
      // Run security checks on session completion
      if (event.type === "session.idle") {
        console.log("🔒 Running security verification...")

        try {
          // Format check
          console.log("📝 Checking code formatting...")
          await $`cargo fmt --all -- --check`

          // Clippy lints
          console.log("🔍 Running Clippy lints...")
          await $`cargo clippy --all-targets --all-features -- -D warnings`

          // Security audit
          console.log("🛡️  Auditing dependencies...")
          try {
            await $`cargo audit`
          } catch {
            console.log("⚠️  Security vulnerabilities found! Run 'cargo audit fix'")
          }

          // Tests
          console.log("🧪 Running tests...")
          await $`cargo test --all`

          console.log("✅ Security checks passed!")
        } catch (error) {
          console.error("❌ Security checks failed:", error.message)
          throw error
        }
      }
    },
  }
}