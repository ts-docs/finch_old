
# Template helper

This helper compiles another template with the given data.

**Syntax:**
```
{{#template templateName templateData /}}
```

*No-body helper*

## Example

```js
Finch.addTemplate("user", "<span>{{number}}. {{name}}</span>");

Finch.addTemplate("users", `
    <ul>
    {{#each users user}} 
        <li>{{#template "user" user /}}</li>
    {{/}}
    </ul>
`);

Finch.compile("users", {
    users: [{name: "Google", number: 0}, {name: "Hidden", number: 10}, {name: "Zoroark", number: 68}]
});
```