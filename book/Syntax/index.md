
# Syntax

Finch's syntax is very similar to handlebar's, but there are a few major differences, which are meant to make the syntax cleaner and simpler. 

## Differences

### `this`

There's no `this` in finch, and there's also no `path` expressions. To access a member in an object, use the dot notation (`obj.member`). You cannot use `/`, and you also cannot go back (`../`)

### Helper syntax

**Handlebars:**
```
{{#if value}}
  ...content...
{{else}}
  ...otherwise...
{{/if}}
```

**Finch:**
```
{{#if value}}
    ...content...
{{/#else}}
    ...otherwise...
{{/}}
```

The handlebars syntax here is quite weird and ambiguous, finch has a simple **block followup** system which allows for easy chaining of blocks, more on that later.

### Partials

Finch has no partials, instead templates act as partials directly. Templates can be rendered within other templates with the `template` helper.
