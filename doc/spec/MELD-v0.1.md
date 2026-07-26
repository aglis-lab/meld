# Meld Format Specification

This is the format file specification will be used as by the compiler to create highly flexible template bytecode.

## Import Syntax

The import syntax is similar with what typescript and javascript have.
Meld is exported as a default export like what svelte and vue do.
To import it we need to use `---` follow by what we need to import and then `---` at the end.

### Nested Meld File

**Source File**

```html
---
import Header from '../components/Header.meld';
---

<!-- This Code is call other meld file -->
<!-- It's will flatten away at compile to for maximum performance at runtime -->
<!-- You also can pass the data -->
<header user="{{user}}"></header>
```

**Target File**

```html
<!-- <Header user={{user}}></Header> -->
<nav>
  <ul>
    <li>About</li>
    <!-- Other nav -->
  </ul>
</nav>
```

### Island File

**Source File**

```html
---
import Counter from '../components/counter.svelte';
---

<!-- You also can inject the data into client side components -->
<!-- It's will rendered as an island component-->
<Counter title={{"Just a regular title"}}></Counter>

```

**Target File**

It's will transpiled/compiled into a simple working templates which can be transpiled into TEF Bytecode Template

```html
<!-- <Counter initialCounter={{user.initialCounter}}></Counter> -->
<meld-island id="counter-svelte" data-framework="svelte"></meld-island>
<script type="application/json" data-for="counter-svelte">
  { "title": "Counter Title" }
</script>

<!-- This is an entry js script which call the runtime which will be executed at client side -->
<!-- This code will be transpiled at compiled time by the vite plugin.  -->
<script>
  import { mount } from "svelte";
  import CounterComponent from '../components/counter.svelte';

  function createCounter(target: HTMLElement, props: {}) {
    return mount(CounterComponent, { target, props });
  }

  function attachElement(elementId: string) {
    const el = document.getElementById(elementId);
    if (!el) {
        return;
    }

    const dataScript = document.querySelector(`script[data-for="${elementId}"]`);
    let props = {};

    if (dataScript?.textContent) {
        try {
            const parsed = JSON.parse(dataScript.textContent);
            if (parsed && typeof parsed === "object") {
                props = parsed;
            }
        } catch (e) {
            console.error("Failed to parse props:", e);
        }
    }

    try {
        createCounter(el, props);
    } catch (error) {
        console.error("Failed to create component:", error);
        console.error("Props were:", props);
    }
  }

  attachElement("counter-svelte");
</script>
```
