import { z } from 'zod'

export const PostSchema = z.object({
  id: z.string().uuid(),
  title: z.string(),
  slug: z.string(),
})

export const CategorySchema = z.object({
  id: z.string().uuid(),
  title: z.string(),
  slug: z.string(),
})

export type Post = z.infer<typeof PostSchema>
export type Category = z.infer<typeof CategorySchema>
