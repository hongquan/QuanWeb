import { z } from 'zod'

export const PaginationLinksSchema = z.object({
  next: z.string().nullable(),
  prev: z.string().nullable(),
})

export const ObjectListResponseSchema = z.object({
  count: z.number(),
  links: PaginationLinksSchema,
  objects: z.array(z.record(z.unknown())),
})

export type PaginationLinks = z.infer<typeof PaginationLinksSchema>
export type ObjectListResponse = z.infer<typeof ObjectListResponseSchema>
