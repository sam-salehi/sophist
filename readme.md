
# Sophist
## motivation
Keeping track of files can be challenging—especially as directories grow, files move, names change, or things get deleted without notice. Traditional tools rely on exact paths or filenames, which makes it easy to lose track of content after even small changes.

Sophist solves this by continuously watching files for movements, renames, edits, and deletions. It generates semantic embeddings so you can search by meaning—not just names—making it easier to find what you're looking for, even if it's no longer where you left it.


## commands
To see in action, run 'sophist \<command> with any of the commands below.'

```
init: Setup storage. Bootup daemon. Pass API keys. Install dependencies.
shutdown: Kill daemon.
watch <relative_path>: Begin stalking a file. Add embedding to storage.
abandon: Stop stalking file for movement. Remove its embedding from database.
search: Begin generating search query.
```

## dependencies

## macOS setup
To setup on a macOS device run the following:
```
git clone https://github.com/sam-salehi/sophist.git
cargo install --path .
export PATH="$HOME/.cargo/bin:$PATH"
```

## linux setup
For a linux device run:
```
```

## overview in action
