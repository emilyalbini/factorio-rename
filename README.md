# Rename Factorio player name in save files

> [!CAUTION]
>
> ***This tool has a non-trivial chance of corrupting your save files.*** Always
> make a backup before using it, and keep the backup around for a few hours of
> play before discarding it.

Factorio stores your player name in the save file itself, and while changing
your name is supported (either through Steam, the Factorio website, or the
settings), the new name you chose only applies to **new** save files. That's
because Factorio stores the player name in the save file itself, and doesn't
offer any feature to retroactively change that name.

This tool, tested on Factorio 2.1 on both vanilla and modded save files,
modifies existing saves to replace your name.

## Caveats and limitations

The format of Factorio save files is largely undocumented, and this tool
attempts to do a blind search-and-replace inside of them. There are multiple
known ways this could go wrong:

* To avoid corrupting the save file, the new name must be equal or lower in
  length than the old name.

* If your old name matches the name of any entity in the game, the tool will
  *also* change the name of that entity, which will result in the entity being
  deleted when you load the game. Sorry, miss `fish`.

* If your old name is `player`, the save file will probably load, but then crash
  at some point in the future as Factorio's runs random consistency checks.

* If your old name is too short, the search-and-replace will undoubtedly find
  other binary data in the save file that *looks* like your old name and replace
  it, which will almost certainly corrupt your save file.

As said before, **make a backup before running the tool.** Unless the Factorio
developers implement a native way to change your name, or someone fully reverse
engineers the save file format, changing your name will be risky.

## Usage

Install [the Rust compiler][rust] on your system, clone this repository, and
then run:

```
cargo run OLD_SAVE_FILE NEW_SAVE_FILE OLD_USERNAME NEW_USERNAME
```

For example:

```
cargo run ~/.factorio/saves/game.zip ~/.factorio/saves/renamed.zip ferris corro
```

## Implementation details

The Factorio save file format is largely undocumented. Existing efforts by the
community to reverse-engineer the save file format focused only on the level
metadata and the mod settings, but the player name isn't stored in any of them.
Instead, it's stored in the chunk file of the chunk currently containing your
player character in the game world.

The chunk files (`level.dat{NUMBER}` in the save archive) are zlib-compressed
binary blobs of data, and they don't appear to be using any known serialization
format. The player name is stored in one of those files, prefixed by an 8-bit
length field. This tool iterates through all the chunk files, finds all
occurrences of the name, and replace them.

To avoid changing the overall length of the save file, if the new name is
shorter it's paddded by null bytes.

[rust]: https://rust-lang.org/tools/install/
