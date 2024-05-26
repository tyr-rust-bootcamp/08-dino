import { execute } from './lib.ts';

async function main() {
  console.log('Executing main');
  console.log(await execute('world'));
}

export default main;
