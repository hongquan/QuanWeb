import { Ok, Error } from "./gleam.mjs"

/**
 * Go up from the passed element to the form
 * and build FormData.
 */
export function getUpFormData(select) {
  const form = select.closest("form")
  if (!form) {
    return []
  }
  return Array.from(new FormData(form).entries())
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

export function showDialog(selector) {
  const dialog = document.querySelector(selector)
  if (dialog) {
    dialog.showModal()
    return true
  }
  console.warn(`${selector} is not found!`)
  return false
}

export function submitForm(form) {
  form.submit()
}

export function getFormData(form) {
  return Array.from(new FormData(form).entries())
}
