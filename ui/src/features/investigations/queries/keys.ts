export const investigationKeys = {
  all: ["investigations"] as const,
  lists: () => [...investigationKeys.all, "list"] as const,
  list: () => [...investigationKeys.lists()] as const,
  details: () => [...investigationKeys.all, "detail"] as const,
  detail: (id: string) => [...investigationKeys.details(), id] as const,
  pinOptions: () => [...investigationKeys.all, "pin-options"] as const,
}
