import { z } from 'zod'

export const PresentationSchema = z.object({
  id: z.string().uuid().nullable().default(null),
  title: z.string().default(''),
  url: z.string().default(''),
  event: z.string().nullable().default(null),
})

export const BookAuthorSchema = z.object({
  id: z.string().uuid().nullable().default(null),
  name: z.string().default(''),
})

export const BookSchema = z.object({
  id: z.string().uuid().nullable().default(null),
  title: z.string().default(''),
  download_url: z.string().nullable().default(null),
  author: BookAuthorSchema.nullable().default(null),
})

export type Presentation = z.infer<typeof PresentationSchema>
export type BookAuthor = z.infer<typeof BookAuthorSchema>
export type Book = z.infer<typeof BookSchema>
