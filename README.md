[![Crate](https://img.shields.io/crates/v/sales.svg)](https://crates.io/crates/sales)
[![Docs](https://docs.rs/sales/badge.svg)](https://docs.rs/sales)
![CI](https://github.com/bitfield/sales/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/bitfield/sales/actions/workflows/audit.yml/badge.svg)
![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

The `sales` crate provides a command-line tool, and library, for reporting and aggregating sales data (for example, from CSV files).

# Installation

```sh
cargo install sales
```

# Usage

Download your sales data for the relevant period to a file, for example, `data.csv`, and run:

```sh
sales data.csv
```

Example output:

```txt
Product / Group                                       Units      Revenue
------------------------------------------------------------------------
Go mentoring                                             11      3134.45
Code For Your Life                                        2        79.90
For the Love of Go (2023)                                 1        39.95
For the Love of Go: Video/Book Bundle (2023 edition)      1        74.95
The Power of Go: Tests                                    1        39.95
The Power of Go: Tools                                    1        39.95
------------------------------------------------------------------------
Total                                                    17      3409.15
```

# Sorting by revenue

The default sorting is by unit sales, descending, and then alphabetically. To sort by revenue instead, use the `--revenue` flag:

```sh
sales --revenue data.csv
```

```txt
Product / Group                                       Units      Revenue
------------------------------------------------------------------------
Go mentoring                                             11      3134.45
Code For Your Life                                        2        79.90
For the Love of Go: Video/Book Bundle (2023 edition)      1        74.95
For the Love of Go (2023)                                 1        39.95
The Power of Go: Tests                                    1        39.95
The Power of Go: Tools                                    1        39.95
------------------------------------------------------------------------
Total                                                    17      3409.15
```

# Grouping related products

To aggregate the sales data for a group of related products, create a group specification file with the following format:

```txt
GROUP_NAME | GROUP_REGEX
```

A simple example, which groups two sets of products by matching shared substrings in their names:

```txt
For the Love of Go | For the Love
Power of Go Series | The Power of Go
```

Use the `--groups` flag to apply this group specification, and rerun the tool to see the result:

```sh
sales --groups groups.txt data.csv
```

```txt
Product / Group     Units      Revenue
--------------------------------------
Go mentoring           11      3134.45
Code For Your Life      2        79.90
For the Love of Go      2       114.90
Power of Go Series      2        79.90
--------------------------------------
Total                  17      3409.15
```

# Input formats

`sales` can interpret CSV data produced by Squarespace, Gumroad, and similar platforms. It reads only the product name, price, and quantity columns.

* The product name column heading should be "Lineitem name" or "Item Name".

* The product price column heading should be "Lineitem price" or "Item Price ($)".

* The product quantity column heading should be "Lineitem quantity" or "Quantity".
