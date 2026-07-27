// 내 업무 data access — the caller's personal work: their action-inbox (assigned
// items, same REAL endpoint the overview uses) and their private todo list
// (/api/v1/me/todos, full CRUD). All person-scoped server-side from the JWT.

import type { components } from "@maintenance/api-client-ts";

import type { ConsoleApiClient } from "../../../api/client";
import type { ActionInboxResponse } from "../overview/overviewModel";

export type TodoSummary = components["schemas"]["TodoSummary"];
export type MyWorkbenchResponse = components["schemas"]["MyWorkbenchResponse"];
export type WorkbenchCalendarItem = components["schemas"]["WorkbenchCalendarItem"];
export type CalendarEventResponse = components["schemas"]["CalendarEventResponse"];

/** Calendar writes remain owned by collaboration.  My Work only supplies the
 * personal scheduling intent and then re-reads its bounded aggregate. */
export interface PersonalCalendarEventInput {
  title: string;
  startsAt: string;
  endsAt: string;
}

export type ActionInboxPage = ActionInboxResponse;

export interface MyWorkApi {
  loadInbox(cursor?: string): Promise<ActionInboxPage>;
  loadTodos(includeDone: boolean): Promise<TodoSummary[]>;
  createTodo(text: string): Promise<void>;
  setTodoDone(id: string, done: boolean): Promise<void>;
  deleteTodo(id: string): Promise<void>;
  loadWorkbench(): Promise<MyWorkbenchResponse>;
  createPersonalCalendarEvent(input: PersonalCalendarEventInput): Promise<CalendarEventResponse>;
}

export function createMyWorkApi(client: ConsoleApiClient): MyWorkApi {
  return {
    loadInbox: async (cursor) => {
      const { data } = await client.GET("/api/v1/me/action-inbox", {
        params: { query: { limit: 200, cursor } },
      });
      if (!data) throw new Error("action-inbox failed");
      if (data.next_cursor && data.next_cursor === cursor) {
        throw new Error("action-inbox cursor did not advance");
      }
      return data;
    },
    loadTodos: async (includeDone) => {
      const { data } = await client.GET("/api/v1/me/todos", {
        params: { query: { include_done: includeDone, limit: 100 } },
      });
      if (!data) throw new Error("todos failed");
      return data.items;
    },
    createTodo: async (text) => {
      const { error } = await client.POST("/api/v1/me/todos", {
        body: { text, scopes: [], links: [] },
      });
      if (error) throw new Error("create todo failed");
    },
    setTodoDone: async (id, done) => {
      const { error } = await client.POST("/api/v1/me/todos/{todoId}/done", {
        params: { path: { todoId: id } },
        body: { done },
      });
      if (error) throw new Error("set todo done failed");
    },
    deleteTodo: async (id) => {
      const { error } = await client.DELETE("/api/v1/me/todos/{todoId}", {
        params: { path: { todoId: id } },
      });
      if (error) throw new Error("delete todo failed");
    },
    loadWorkbench: async () => {
      const { data } = await client.GET("/api/v1/me/workbench", {
        params: { query: {} },
      });
      if (!data) throw new Error("workbench failed");
      return data;
    },
    createPersonalCalendarEvent: async ({ title, startsAt, endsAt }) => {
      const { data } = await client.POST("/api/v1/collaboration/calendar/events", {
        body: {
          scope_type: "PERSONAL",
          title,
          starts_at: startsAt,
          ends_at: endsAt,
          all_day: false,
        },
      });
      if (!data) throw new Error("calendar event failed");
      return data;
    },
  };
}
