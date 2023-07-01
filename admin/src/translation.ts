import { FluentBundle } from '@fluent/bundle'
import { createFluentVue } from 'fluent-vue'

import { Language } from './models'

const enBundle = new FluentBundle('en')
const viBundle = new FluentBundle('vi')

export const fluent = createFluentVue({
  bundles: [enBundle],
})

export function preferLocale(lang: Language) {
  if (lang === Language.EN) {
    fluent.bundles = [enBundle]
  } else {
    fluent.bundles = [viBundle]
  }
}
