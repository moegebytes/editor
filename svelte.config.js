export default {
  compilerOptions: {
    runes: true,
    warningFilter: (warning) =>
      !["a11y_no_static_element_interactions", "a11y_click_events_have_key_events"].includes(warning.code),
  },
};
