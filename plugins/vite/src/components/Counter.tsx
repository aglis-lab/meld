import { Hello } from "./Hello";
import { useState } from "react";

export default function Counter({ great }: { great: string }) {
  const [counter, setCounter] = useState(0);

  return (
    <>
      <h2>
        Counter Click here{" "}
        <button onClick={() => setCounter(counter + 1)}>Increment</button>
      </h2>
      <p>Counter is {counter}</p>
      <p>Great: {great}</p>
      <Hello></Hello>
    </>
  );
}
