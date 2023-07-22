import type { RequireAtLeastOne } from 'type-fest'
import { D } from '@mobily/ts-belt'

import { WithCategories } from '@/models/blog'
import { Book } from '@/models/minors'

export function transformPostForPosting(origData: RequireAtLeastOne<WithCategories, 'categories'> & { created_at: Date } ) {
  const categoriesIds = origData.categories.map((c) => c.id)
  const stripped = D.deleteKeys(origData, ['id', 'categories', 'created_at'])
  const postData = D.set(stripped, 'categories', categoriesIds)
  return postData
}

export function transformBookForPosting(origData: Book) {
  const authorId = origData.author ? origData.author.id : null
  const stripped = D.deleteKeys(origData, ['id', 'author'])
  const postData = D.set(stripped, 'author', authorId)
  return postData
}
