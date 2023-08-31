import type { RequireAtLeastOne } from 'type-fest'
import { D } from '@mobily/ts-belt'

import { WithCategories } from '@/models/blog'
import { Book } from '@/models/minors'
import { User } from '@/models/user'

export function transformPostForPosting(origData: RequireAtLeastOne<WithCategories, 'categories'> & { author: User | null, created_at: Date } ) {
  const categoriesIds = origData.categories.map((c) => c.id)
  const authorId = origData.author ? origData.author.id : null
  const stripped = D.deleteKeys(origData, ['id', 'categories', 'author', 'created_at'])
  const postData = D.merge(stripped, { author: authorId, categories: categoriesIds })
  return postData
}

export function transformBookForPosting(origData: Book) {
  const authorId = origData.author ? origData.author.id : null
  const stripped = D.deleteKeys(origData, ['id', 'author'])
  const postData = D.set(stripped, 'author', authorId)
  return postData
}
