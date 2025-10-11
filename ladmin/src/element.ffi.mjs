import { Ok, Error } from "./gleam.mjs"

/**
 * Go up from the passed element to the form
 * and build FormData.
 * @param {HTMLElement} select - The element to start searching from
 * @returns {Array<[string, FormDataEntryValue]>} Array of form data entries
 */
export function getUpFormData(select) {
  const form = select.closest("form")
  if (!form) {
    return []
  }
  return Array.from(new FormData(form).entries())
}

/**
 * Get the value of a form field by name
 * @param {HTMLElement} elm - The element to start searching from
 * @param {string} name - The name of the form field
 * @returns {Error<undefined> | Ok<string>} Result containing the field value or error
 */
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

/**
 * Show a dialog element using its selector
 * @param {string} selector - CSS selector for the dialog element
 * @returns {boolean} True if dialog was shown, false otherwise
 */
export function showDialog(selector) {
  const dialog = document.querySelector(selector)
  if (dialog) {
    dialog.showModal()
    return true
  }
  console.warn(`${selector} is not found!`)
  return false
}

/**
 * Submit a form programmatically
 * @param {HTMLFormElement} form - The form element to submit
 * @returns {void}
 */
export function submitForm(form) {
  form.submit()
}

/**
 * Get form data as an array of entries
 * @param {HTMLFormElement} form - The form element
 * @returns {Array<[string, FormDataEntryValue]>} Array of form data entries
 */
export function getFormData(form) {
  return Array.from(new FormData(form).entries())
}

/**
 * Show a confirmation dialog to the user
 * @param {string} message - The message to display in the confirmation dialog
 * @returns {boolean} True if user confirmed, false otherwise
 */
export function confirm(message) {
  return window.confirm(message)
}
