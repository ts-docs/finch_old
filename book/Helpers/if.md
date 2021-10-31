
# If helper

**Syntax:**

```
{{#if condition}} 
   ...content...
{{/}}
```

**Possible followups:**

*if:*
Will be checked if the condition of the previous if statement returns false. Acts as an `else if`.
```
{{/#if condition}}
   ...content...
{{/}}
```

*else*
```
{{/#else}}
    ...content...
{{/}}
```

## Example

```js
Finch.addTemplate("example", `
<div>
    {{#if hasFlag("admin") }} 
        <p>Admin</p>
    {{/#else}}
        <p>Regular User</p>
    {{/}}
</div>
`);


console.log(Finch.compile("example", {
    hasFlag: (flag) => {
        if (flag === "admin") return true;
    }
}));
```