import type { RequireAtLeastOne } from 'type-fest'
import { D } from '@mobily/ts-belt'

import { WithCategories } from '@/models/blog'

export function transformPostForPosting(origData: RequireAtLeastOne<WithCategories, 'categories'> & { created_at: Date } ) {
  const categoriesIds = origData.categories.map((c) => c.id)
  const stripped = D.deleteKeys(origData, ['id', 'categories', 'created_at'])
  const postData = D.set(stripped, 'categories', categoriesIds)
  return postData
}
