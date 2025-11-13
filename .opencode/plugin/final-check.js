export const FinalCheckPlugin = async ({ project, client, $, directory, worktree }) => {
  console.log("🏁 Final check plugin initialized!")

  return {
    event: async ({ event }) => {
      // Run final verification on session completion
      if (event.type === "session.idle") {
        console.log("🏁 Running final session verification...")

        try {
          // Check if any Rust files were modified
          const modifiedFiles = await $`git diff --name-only`
          const hasRustChanges = modifiedFiles.stdout?.includes('.rs')

          if (hasRustChanges) {
            console.log("📊 Verifying Rust code quality...")

            // Final build check
            console.log("🔨 Running final build...")
            await $`cargo build --all`

            // Final test check
            console.log("🧪 Running final tests...")
            await $`cargo test --all --quiet`
          }

          // Check for uncommitted changes to Cargo.lock
          const cargoLockChanged = modifiedFiles.stdout?.includes('Cargo.lock')
          if (cargoLockChanged) {
            console.log("📦 Cargo.lock was modified. Remember to commit it.")
          }

          console.log("✅ Session verification complete")
        } catch (error) {
          console.error("❌ Final checks failed:", error.message)
          throw error
        }
      }
    },
  }
}