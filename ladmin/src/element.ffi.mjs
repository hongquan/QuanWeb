import { Ok, Error } from "./gleam.mjs"

export function getFormData(select) {
  const form = select.closest("form")
  if (!form) {
    return []
  }
  return Array.from(form.elements).map(e => [e.name, e.value])
}

export function getFormFieldValue(elm, name) {
  const form = elm.closest("form")
  if (!form) {
    return new Error(undefined)
  }
  const field = form.elements[name]
  if (!field) {
    return new Error(undefined)
  }
  return new Ok(field.value)
}
