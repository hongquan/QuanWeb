import type { RequireAtLeastOne } from 'type-fest'
import { D } from '@mobily/ts-belt'

import { WithCategories } from '@/models/blog'

export function transformPostForPosting(origData: RequireAtLeastOne<WithCategories, 'categories'>) {
  const categoriesIds = origData.categories.map((c) => c.id)
  const stripped = D.deleteKey(origData, 'categories')
  const postData = D.set(stripped, 'categories', categoriesIds)
  return postData
}
