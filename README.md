# cjval: schema-validation of CityJSON + CityJSONSeq datasets

[![crates.io](https://img.shields.io/crates/v/cjval.svg)](https://crates.io/crates/cjval)
[![GitHub license](https://img.shields.io/github/license/cityjson/cjval)](https://github.com/cityjson/cjval/blob/main/LICENSE)

A Rust library and binaries to validate the syntax of CityJSON objects (CityJSON + [CityJSONSeq](https://www.cityjson.org/cityjsonseq)).

It validates against the [CityJSON schemas](https://www.cityjson.org/schemas) and additional functions have been implemented (because these checks cannot be expressed with [JSON Schemas](https://json-schema.org/)).

The following error checks are performed:

  1. *JSON syntax*: is it a valid JSON object?
  1. *CityJSON schemas*: validation against the schemas (CityJSON v1.0 + v1.1 + v2.0)
  1. *Extension schemas*: validate against the extra schemas if there's an [Extension](https://www.cityjson.org/extensions/) (those are automatically fetched from a URL)
  1. *parents_children_consistency*: if a City Object references another in its `"children"`, this ensures that the child exists. And that the child has the parent in its `"parents"`
  1. *wrong_vertex_index*: checks if all vertex indices exist in the list of vertices
  1. *semantics_array*: checks if the arrays for the semantics in the geometries have the same shape as that of the geometry and if the values are consistent
  1. *textures*: checks if the texture arrays are coherent (if the referenced vertices exist and if the texture exists)
  1. *materials*: checks if the arrays for the materials are coherent with the geometry objects and if the material exists

It also verifies the following, these are not errors but warnings since the file is still considered valid and usable, but they can make the file larger and some parsers might not understand all the properties:

  1. *extra_root_properties*: if CityJSON has extra root properties, these should be documented in an Extension. If not this warning is returned
  1. *duplicate_vertices*: duplicated vertices in `"vertices"` are allowed, but they take up space and reduce the explicit topological relationships in the file. If there are any, [cjio](https://github.com/cityjson/cjio) has the operator `clean` to fix this automatically.
  1. *unused_vertices*: vertices that are not referenced in the file, they take extra space. If there are any, [cjio](https://github.com/cityjson/cjio) has the operator `clean` to fix this automatically.


## A Rust library + 2 binaries

`cjval` is a Rust library, and has 2 different binaries:

  1. `cjval` to validate a CityJSON file or a CityJSONSeq stream (it downloads Extensions automatically if the file contains any)
  2. `cjvalext` to validate a [CityJSON Extension file](https://www.cityjson.org/specs/#the-extension-file)


## Installation/compilation

### To install the binaries on your system easily

1. install the [Rust compiler](https://www.rust-lang.org/learn/get-started)
2. `cargo install cjval --features build-binary`

### To compile the project (and eventually modify it)

1. install the [Rust compiler](https://www.rust-lang.org/learn/get-started)
2. `git clone https://github.com/cityjson/cjval.git`
3. `cargo build --release --features build-binary` (this will ensure the binaries are compiled too)
4. `./target/release/cjval myfile.json`


## Web application

The code is used at [https://validator.cityjson.org](https://validator.cityjson.org), it is compiled as a WebAssembly ([WASM code here](https://github.com/cityjson/cjval_wasm)) and a simple GUI was built.


## CLI Usage

### For CityJSON files

The [CityJSON schemas](https://www.cityjson.org/schemas/) are packaged with the binary, so it suffices to:
  
```sh
cjval myfile.city.json --verbose
```

(the latest schemas of a X.Y version will be automatically fetched)

`--verbose` is used to get a detailed report per error check.

If the file contains one or more [Extensions](https://www.cityjson.org/extensions/), eg:

```json
{
  "type": "CityJSON",
  "version": "2.0",
  "extensions":
  {
    "Potato":
    {
      "url": "https://www.cityjson.org/extensions/potato.ext.json",
      "version": "1.0"
    }
  }
...  
```

then `cjval` will download the Extension schema files automatically.

If instead you want to use your own local Extension schema(s), you can pass them as argument with the `-e` flag and this will overwrite the automatic download:

```sh
cjval myfile.city.json -e ./myextensions/generic.ext.json
```

### For CityJSONSeq

To validate a stream of [CityJSONFeature](https://www.cityjson.org/cityjsonseq/), you need to 'cat' the file:

```sh
cat mystream.city.jsonl | cjval --verbose
```

Or you can use [cjseq](https://github.com/cityjson/cjseq) to generate the stream from a CityJSON file:

```sh
cjseq cat -f myfile.city.json | cjval --verbose
```

and you'll get a short report per line (which is one `CityJSON` followed by several `CityJSONFeature`).

`--verbose` is used to get a detailed report per line, if not used then only lines with errors are reported.


## Contributors

- [@hugoledoux](https://github.com/hugoledoux/)
- [@josfeenstra](https://github.com/josfeenstra/) (started the project for an [MSc Geomatics course at TU Delft](https://3d.bk.tudelft.nl/courses/geo5010/), [original code](https://github.com/josfeenstra/cjval))
