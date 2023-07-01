import { z } from 'zod'

export const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
})

export type User = z.infer<typeof UserSchema>
