rush [![Build Status](https://travis-ci.org/ordovicia/rush.svg?branch=master)](https://travis-ci.org/ordovicia/rush)
====

Toy shell implemented in Rust.

## Features

* Redirection
    * Input: `command < file`
    * Truncate: `command > file`
    * Append: `command >> file`
    * Composition of the above: e.g. `command < file >> file`
* Pipe: `command | command`
    * Combination with redirections: `command < file | command | command > file`
* Built-in commands
    * `cd`
