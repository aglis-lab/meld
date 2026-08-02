import type { App } from "vue";
import CounterVue from "./CounterVue.vue";

interface CounterProps {
  title?: string;
}

async function createCounter(target: HTMLElement, props: CounterProps = {}) {
  const { createApp } = await import("vue");

  const app: App = createApp(CounterVue, {
    title: props.title || "Vue Counter",
  });

  return app.mount(target);
}

function attachElement(elementId: string) {
  const el = document.getElementById(elementId);
  if (!el) {
    return;
  }

  const dataScript = document.querySelector(`script[data-for="${elementId}"]`);
  let props: CounterProps = {};

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

  createCounter(el, props).catch((error) => {
    console.error("Failed to create component:", error);
    console.error("Props were:", props);
  });
}

attachElement("counter-vue");
