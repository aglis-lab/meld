import { createElement } from "react";
import { createRoot } from "react-dom/client";
import Counter from "./Counter";

interface CounterProps {
  great?: string;
}

function createCounter(target: HTMLElement, props: CounterProps = {}) {
  createRoot(target).render(
    createElement(Counter, { great: props.great || "React Counter" }),
  );
}

function attachElement(elementId: string) {
  const el = document.getElementById(elementId);
  if (!el) {
    return;
  }

  const dataScript = document.querySelector(`script[data-for="${elementId}"]`);
  const props = dataScript ? JSON.parse(dataScript.textContent || "{}") : {};
  createCounter(el, props);
}

attachElement("counter-react");
