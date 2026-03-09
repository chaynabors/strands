import { run, ROOT } from "../run.js";

export interface ExampleOptions {
  rs?: boolean;
  py?: boolean;
  ts?: boolean;
  kt?: boolean;
  java?: boolean;
}

export async function example(
  name: string,
  opts?: ExampleOptions,
): Promise<void> {
  if (opts?.py) {
    run(`.venv/bin/python examples/${name}.py`, {
      cwd: `${ROOT}/strands-py`,
    });
  } else if (opts?.ts) {
    run(`npm start`, {
      cwd: `${ROOT}/strands-ts/examples/${name}`,
    });
  } else if (opts?.kt) {
    run(`./strands-kt/gradlew -p strands-kt :examples-kt:run`);
  } else if (opts?.java) {
    run(`./strands-kt/gradlew -p strands-kt :examples-java:run`);
  } else {
    run(`cargo run -p strands --example ${name}`);
  }
}
