# cjval: a validator for CityJSON files

[![crates.io](https://img.shields.io/crates/v/cjval.svg)](https://crates.io/crates/cjval)
[![GitHub license](https://img.shields.io/github/license/cityjson/cjval)](https://github.com/cityjson/cjval/blob/main/LICENSE)


A validator for [CityJSON files](https://cityjson.org), it validates against the [CityJSON schemas](https://www.cityjson.org/schemas) and additional functions have been implemented (because these can't be expressed with [JSON Schema](https://json-schema.org/)).


## What is validated exactly?

Schema validation means: is the syntax of the file OK?
The following is performed by `cjval`:

  1. *JSON syntax*: is the file a valid JSON file?
  1. *CityJSON schemas*: validation against the schemas (CityJSON v1.0 or v1.1)
  1. *Extension schemas*: validate against the extra schemas if there's an Extension in the input file 
  1. *parent_children_consistency*: if a City Object references another in its `children`, this ensures that the child exists. And that the child has the parent in its `parents`
  1. *wrong_vertex_index*: checks if all vertex indices exist in the list of vertices
  1. *semantics_array*: checks if the arrays for the semantics in the geometries have the same shape as that of the geometry and if the values are consistent

It also verifies the following, these are not errors since the file is still considered valid and usable, but they can make the file larger and some parsers might not understand all the properties:

  1. *extra_root_properties*: if CityJSON has extra root properties, these should be documented in an Extension. If not this warning is returned
  1. *duplicate_vertices*: duplicated vertices in `vertices` are allowed, but they take up spaces and decreases the topological relationships explicitly in the file. If there are any, [cjio](https://github.com/cityjson/cjio) has the operator `clean` to fix this automatically.
  1. *unused_vertices*: vertices that are not referenced in the file, they take extra space. If there are any, [cjio](https://github.com/cityjson/cjio) has the operator `clean` to fix this automatically.


## Installation/compilation

### To install the binary on your system easily

1. install the [Rust compiler](https://www.rust-lang.org/learn/get-started)
2. `cargo install cjval --features build-binary`


### To compile the project (and eventually modify it)

1. install the [Rust compiler](https://www.rust-lang.org/learn/get-started)
2. `git clone https://github.com/cityjson/cjval.git`
3. `cargo build --release --features build-binary`
4. `./target/release/cjval myfile.json`


## Web application

The code is use at [https://validator.cityjson.org](https://validator.cityjson.org), that is it is compiled as WebAssembly ([WASM code here](https://github.com/cityjson/cjval_wasm)).


## CLI Usage

The [CityJSON schemas](https://www.cityjson.org/schemas/) are built-in the binary, so it suffices to:

    $ cjval myfile.city.json

(the latest schemas of a X.Y version will be automatically fetched)

If the file contains one or more [Extensions](https://www.cityjson.org/extensions/), eg:

```json
{
  "type": "CityJSON",
  "version": "1.1",
  "extensions":
  {
    "Generic":
    {
      "url": "https://www.cityjson.org/extensions/generic.ext.json",
      "version": "1.0"
    }
  }
...  
```

then `cjval` will fetch/download automatically the schema(s).

If instead you want to use your own local Extension schema(s), you can pass them as argument with the argument `-e`:

    $ cjval myfile.city.json -e ./myextensions/generic.ext.json


## Contributors

- [@hugoledoux](https://github.com/hugoledoux/)
- [@josfeenstra](https://github.com/josfeenstra/) (started the project for a [course at TU Delft](https://3d.bk.tudelft.nl/courses/geo5010/), [original code](https://github.com/josfeenstra/cjval))