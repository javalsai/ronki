# MrRonki
Same concept as [MrKonqi](https://github.com/Pickzelle/MrKonqi/) but in rust, with some additional stuff:
* Text based commands, such that they're compatible with matrix.
* A setup as simple a possible, no paralload install, no CPP libs, just cargo and the db (also easy to set up).
* Awesome command awareness, each message as it's own environment and inputs/outputs defined, so you can easily connect them via subshells or env assignations.

# Installation
## Requirements
* unix based system, I won't consider supporting anything else
* cargo (nightly)
* a SurrealDB

## Database
Just install SurrealDB and then you can run `surreal start file:.db`, it will use the local directory `.db` to store data, (automatically ignored by git), don't forget to configure the config file accordingly.

---

For anything else like configuration just check the help menu (run the command with `-h`)
