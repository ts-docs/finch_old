# finch

A super fast and efficient template rendering engine for node.js, inspired by **Handlebars**. 

## Building blocks

There are three types of building blocks in finch:

- `Text`, which could be HTML, markdown or just normal text. 
- `Templates`, which are inside the text block (`{{}}`)
- `Expressions`, which are inside the templates. 

## Syntax

### Types of expressions

#### Properties  / Dot Notation

```Hello {{user.name}}, what were you doing on {{date}}```

#### Literals (string, number, boolean, undefined, null)

```{{1}} == {{2}} is {{false}}```

#### Comparisons

```{{user.name == "Google"}}```

Available operators: `==`, `!=`, `>`, `<`, `<=`, `>=`.

#### Logic

`&&` - AND
`||` - OR

#### Function calls

Where `arg1` - `argN` are other expressions.

```{{func(arg1, arg2, argN)}}```


### Function blocks

Where `arg1` - `argN` are expressions.

```handlebars
{{#funcName arg1 arg2 argN}} 

{{/funcName}}
```

**Function piping:**

```handlebars
{{#funcName arg1 arg2 argN}} 

{{/funcName #otherFunc arg1 arg2 argN}}

{{/otherFunc}}
```

#### Built-in function blocks

**if**
```handlebars
{{#if name == "Google"}} 
  <p>You're Google!</p>
{{/if}}
```

**if - else**
```handlebars
{{#if name == "Google"}} 
  <p>You're Google!</p>
{{/#else}}
   <p>You're not Google!</p>
{{/}}
```

**if - elseif**
```handlebars
{{#if name == "Google"}}
    <p>You're Google!</p>
{{/#elseif name == "Admin"}} 
    <p>You're an admin!</p>
{{/#else}} 
    <p>You're {{name}}!</p>
{{/else}}
```

**each**
```handlebars
{{#each names "name"}}
    {{name}}
{{/}}
```

**js**

Allows you to run javascript code. Return a string with text from the code and it'll be rendered. You can access the current object with `self`.

```handlebars
{{#js}}
    return self.name;
{{/}}
```