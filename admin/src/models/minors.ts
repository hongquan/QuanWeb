import { z } from 'zod'

export const PresentationSchema = z.object({
  id: z.string().uuid().nullable().default(null),
  title: z.string().default(''),
  url: z.string().default(''),
  event: z.string().nullable().default(null),
})

export type Presentation = z.infer<typeof PresentationSchema>
