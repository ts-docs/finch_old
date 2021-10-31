
# Each helper

Each iterates over an array, re-compiling it's body on each iteration and concatenating it with a string, which gets returned in the end.

**Syntax:**
```
{{#each array temporaryVariable}}
    ...content...
{{/}}
```

`temporaryVariable` is the current element.

## Example

```js
Finch.addTemplate("example", `
    <ul>
    {{#each users user}}
        <li>{{user.number}}. {{user.name}}</li>
    {{/}}
    </ul>
`);

Finch.compile("example", {
    users: [{name: "Google", number: 0}, {name: "Hidden", number: 10}, {name: "Zoroark", number: 68}]
});
```