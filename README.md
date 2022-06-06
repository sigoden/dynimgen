# postergen

A high performance poster generator.

<!-- #[demo]() -->

## How to use

1. The designer exports the design drawing as an svg file

```svg
<svg>
  <image src="data:image/png;base64,...." /> 
  <image src="data:image/png;base64,...." />
  <text>66666</text>
</svg>
```

2. The engineer edits the svg file, replaces the changed parts with template variables, and generates the svg template `poster1.svg`

```svg
<svg>
  <img src="{{ avatar | fetch }}">
  <img src="{{ invite_url | to_qr }}">
  <text>{{ code }}</text>
</svg>
```

3. Run `postergen`, make sure the svg template is in the `postergen` workdir

```sh
$ ls data
poster1.svg

$ postergen data/
[2022-06-05T14:51:53Z INFO  postergen::generator] Mount `/poster1`
[2022-06-05T14:51:53Z INFO  postergen] Listen on 0.0.0.0:8080
```

4. Visit `http://localhost:8080/poster1?avatar=http://site/avatar.png&invite_url=http://site/invite&code=12345` to view the generated poster

## How it work

1. Extract data from the query part of url
2. Pass data to template engine to generate a new svg file
3. Render the svg to a png and response

## Syntax

**postergen** uses [Tera](https://github.com/Keats/tera) as the template engine. It has a syntax based on [Jinja2](http://jinja.pocoo.org/) and [Django](https://docs.djangoproject.com/en/3.1/topics/templates/) templates.

There are 3 kinds of delimiters and those cannot be changed:

- `{{` and `}}` for expressions
- `{%` or `{%-` and `%}` or `-%}` for statements
- `{#` and `#}` for comments

See the [Tera Documentation](https://tera.netlify.app/docs/#templates) for more information about [control structures](https://tera.netlify.app/docs/#control-structures), [built-ins filters](https://tera.netlify.app/docs/#built-ins), etc.


Custom built-in filters that **postergen** uses:

- `fetch`: Fetch remote resource and encode as data-url
- `to_qr`: Convert text to qrcode

