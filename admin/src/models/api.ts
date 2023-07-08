import { z } from 'zod'

export const PaginationLinksSchema = z.object({
  next: z.string().nullable(),
  prev: z.string().nullable(),
})

export const ObjectListResponseSchema = z.object({
  count: z.number(),
  total_pages: z.number().min(1),
  links: PaginationLinksSchema,
  objects: z.array(z.record(z.unknown())),
})

export const GeneralErrorResponseSchema = z.object({
  message: z.string(),
})

export type PaginationLinks = z.infer<typeof PaginationLinksSchema>
export type ObjectListResponse = z.infer<typeof ObjectListResponseSchema>
export type GeneralErrorResponse = z.infer<typeof GeneralErrorResponseSchema>
