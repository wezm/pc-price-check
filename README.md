pc-price-check
==============

![Screenshot](Screenshot.png)

`pc-price-check` checks the price of PC components on [staticICE] against
reference prices so you can see when they drop/are on special.

Usage
-----

The tool will check the price of each component listed in `parts.toml`. There
is a [sample file] in the repository. Each component entry specifies the
component type, the reference price and the search query to make. You can have
multiple entries for the same component type.

When the tool is run the search query for each component is run and the lowest
price is compared to the reference price and printed to show if it has gone up
or down.

If you pass `-l` the list of components and their reference prices will be
printed.

Installation
------------

    cargo install --git https://github.com/wezm/pc-price-check.git

[staticICE]: https://www.staticice.com.au/
[sample file]: https://github.com/wezm/pc-price-check/blob/main/parts.example.toml
