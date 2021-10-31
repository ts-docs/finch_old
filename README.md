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

## Features

Finch extends (and changes) the handlebars syntax all the while using the native speed of the Rust programming language.

Here is an overview of all the different featues.

### Expressions inside templates

You can compare variables and call javascript functions inside templates!

```js
Finch.addTemplate("test", '{{value1 == value2}}, {{someFn("Hello")}}');

Finch.compile("test", {
    value1: 100,
    value2: "100",
    someFn: (str) => str + " World"
});
// false, Hello World
```

### Built-in and custom helpers

```js
Finch.addTemplate("test", `
    <div>
    {{#each numbers num}}
        {{num + 1}}
    {{/}}
    </div>
`);

Finch.compile("test", {numbers: [1, 2, 3, 4, 5, 6, 7, 8, 9, 0]})
```