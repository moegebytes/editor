export default {
  compilerOptions: {
    runes: true,
    warningFilter: (warning) => !warning.code.startsWith("a11y"),
  },
};
