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

## Syntax

Finch extends the handlebars syntax all the while using the native speed of the Rust programming language.
There are some pretty big differences with the handlebars syntax, though.

