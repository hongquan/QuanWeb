import cookie from 'cookieman'
import ky, { BeforeRequestHook } from 'ky'

export const writeMethods = ['post', 'put', 'patch', 'delete']

export const kyXsrfConfig = {
  hooks: {
    beforeRequest: [
      (request) => {
        if (writeMethods.includes(request.method.toLowerCase())) {
          request.headers.set('X-CSRFToken', cookie.val('csrftoken') || '')
        }
      },
    ] as BeforeRequestHook[],
  },
}

export const kyClient = ky.create(kyXsrfConfig)
