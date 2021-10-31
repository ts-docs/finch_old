# finch

A super fast and efficient template rendering engine for node.js, inspired by **Handlebars**. 

## Usage

Finch is very simple to use: Register a template, then compile it with your own data. Every registered template gets pre-compiled for performance purposes.

```js
const Finch = require("finch");

Finch.addTemplate("hello_world", "Hello {{name}}, welcome to the world of {{world}}");

console.log(Finch.compile("hello_world", {name: "Google", world: "Finch"})); 
// Hello Google, welcome to the world of Finch
```

Check out the [official book](ts-docs.github.io/finch/) for complete examples, guides and features!

## Outlined Features

- All handlebar features, plus...
- ~x3 times faster than `handlebars`.
- Simple, easy and readable templates.
- Bootstrap JS right inside your templates.
- Call JS functions inside templates.
- Use the built-in handlers (each, if) or make your own.
