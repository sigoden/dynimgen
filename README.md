# dynimgen

[![CI](https://github.com/sigoden/dynimgen/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/dynimgen/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/dynimgen.svg)](https://crates.io/crates/dynimgen)

 A self-hosted dynamic image generator.

## How to use


step 1: The designers export the design drawing as an svg file

```svg
<svg>
  <rect />
  <image src="img.png" /> 
  <image src="qr.png" />
  <text>66666</text>
</svg>
```

step 2: The engineers edit the svg file, replace the changed parts with template variables

```
<svg>
  <rect />
  <img src="{{ img | fetch }}">
  <img src="{{ qr | to_qr }}">
  <text>{{ code }}</text>
</svg>
```

step 3: Run `dynimgen`, make sure the svg template is in the `dynimgen` workdir

```sh
$ ls data
poster1.svg

$ dynimgen fixtures/
[2022-06-05T14:51:53Z INFO  dynimgen::generator] Mount `/poster1`
[2022-06-05T14:51:53Z INFO  dynimgen] Listen on 0.0.0.0:8080
```

step 4: Build image url according to the following rules

```
<domain> + <template path> + ? + <template variables>
```

For example:

- domain: http://localhost:8080
- template path: /poster1
- template variables: { img: "https://picsum.photos/250", qr: "dynimgen", code: 12345 }

Build url:

http://localhost:8080/poster1?img=https://picsum.photos/250&qr=dynimgen&code=12345

If you request this url, dynimgen will response a png image.

what dynimgen does:

1. Extract variables from the url query of request
2. Pass variables to template engine to generate a svg
3. Render the svg to a png then response

## Advantages

### Server-side rendering advantages

- No browser compatibility, platform compatibility and other issues
- Code reusability is high, and the poster generation service of h5, applet, and app can be used.
- Can be updated in a timely and convenient manner

### SVG Template Advantages

- Export SVG directly from design software such as PS/AI
- No need to reproduce the implementation with html/canvas/dsl
- Lossless restoration of the design draft without losing any details
- Free use of font styles

### RUST Advantage

- High performance, high concurrency
- Single executable program, cross-platform, easy to deploy

## Install

### With cargo

```
cargo install dynimgen
```

### With docker

```
docker run -v `pwd`/data:/data  -p 8080:8080 --rm -it sigoden/dynimgen /data
```

### Binaries on macOS, Linux, Windows

Download from [Github Releases](https://github.com/sigoden/dynimgen/releases), unzip and add duf to your $PATH.

## Template Engine

**dynimgen** uses [Tera](https://github.com/Keats/tera) as the template engine. It has a syntax based on [Jinja2](http://jinja.pocoo.org/) and [Django](https://docs.djangoproject.com/en/3.1/topics/templates/) templates.

See the [Tera Documentation](https://tera.netlify.app/docs/#templates) for more information about [control structures](https://tera.netlify.app/docs/#control-structures), [built-ins filters](https://tera.netlify.app/docs/#built-ins), etc.


Custom built-in filters that **dynimgen** uses:

### `fetch`

Fetch remote resource and encode as data-url

Example: `{{ img | fetch }}` `{{ img | fetch(timeout=10000) }}`  

### `to_qr`

Convert text to qrcode

Example: `{{ qr | to_qr }}`  `{{ qr | to_qr(bg='#fff', fg='#000') }}` 

## License

Copyright (c) 2022 dynimgen-developers.

dynimgen is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.