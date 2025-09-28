export function getFormData(select) {
  const form = select.closest("form")
  return Array.from(form.elements).map(e => [e.name, e.value])
}
