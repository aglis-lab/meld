import { mount } from "svelte";
import CounterComponent from "./CounterSvelte.svelte";

interface CounterProps {
  title?: string;
}

function createCounter(target: HTMLElement, props: CounterProps = {}) {
  return mount(CounterComponent, { target, props });
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

  try {
    createCounter(el, props);
  } catch (error) {
    console.error("Failed to create component:", error);
    console.error("Props were:", props);
  }
}

attachElement("counter-svelte");
