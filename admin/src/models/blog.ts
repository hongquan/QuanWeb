import { z } from 'zod'

export const CategorySchema = z.object({
  id: z.string().uuid(),
  title: z.string(),
  slug: z.string(),
})

export const WithCategoriesSchema = z.object({
  categories: z.array(CategorySchema),
})

export const PostSchema = z.object({
  id: z.string().uuid(),
  title: z.string(),
  slug: z.string(),
  categories: z.array(CategorySchema),
})

export type Post = z.infer<typeof PostSchema>
export type Category = z.infer<typeof CategorySchema>
export type WithCategories = z.infer<typeof WithCategoriesSchema>
