# Rust Git-Clone Implementation

This project is implemented in Rust and aims to provide a minimalistic clone of some fundamental `git` commands.
This is a challenge from codecrafter.io.

## Features

1. **`git init`**: Initializes a repository by creating a `.git` directory along with its foundational files.
2. **Read Blob**: Extract the content of a blob from your git repository by fetching it from the `.git/objects` directory.
3. **Add Blob**: Introduce a blob to your git repository by leveraging the `git hash-object` command.
4. **`git ls-tree`**: Used for inspecting a tree object.
5. **Write Tree**: Upon invoking the program with `./your_git.sh write-tree`, it writes the entire working directory as a tree object and subsequently prints the 40-character SHA.
6. **`git commit-tree`**: A plumbing command that facilitates the creation of a commit.

## How to Use

1. Ensure you have Rust installed on your system.
2. Clone the repository and navigate to its directory.
3. Use the command `./your_git.sh <command>` where `<command>` can be any of the supported features listed above.
