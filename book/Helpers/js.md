
# JS Helper

The JS helper allows you to execute javascript code and render the return value. Use the `data` variable to access the data that was sent to the template. You cannot use template syntax inside the code.

**Syntax:**

```
{{#js}}
    ...code...
{{/js}}
```

## Example

```js
Finch.addTemplate("example", `
    <div>
        {{js}}
            let res = "";
            if (data.admin) res += "<span>Admin</span>";
            if (data.host) res += "<span>Host</span>";
            return res;
        {{/}}
    </div>
`)

console.log(Finch.compile("example", {
    admin: true,
    host: false
}));
```

## Security concerns

The executed javascript code is not ran or a VM or anything similar. You should **never** compile untrusted templates (for example, templates received from users) while having the `js` helper enabled. To disable the `js` helper, run the following function:

```js
Finch.removeHelper("js");
```