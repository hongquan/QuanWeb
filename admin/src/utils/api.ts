import { z } from 'zod'
import { HTTPError } from 'ky'
import { D } from '@mobily/ts-belt'
import { toast } from 'vue-sonner'

export async function handleApiError(e: unknown): Promise<Record<string, string>> {
  if (e instanceof HTTPError) {
    const resp = await e.response.json()
    const result1 = z.record(z.string()).safeParse(resp.fields)
    if (result1.success) {
      toast.error('Validation error')
      return D.fromPairs(D.toPairs(result1.data))
    }
    const result2 = z.string().safeParse(resp.message)
    if (result2.success) {
      toast.error(result2.data)
      return {}
    }
  }
  console.debug(e)
  return {}
}
