//! This face's slice of the published OpenAPI contract.
//!
//! Bodies are verbatim from `backend/openapi/openapi.yaml`; `console_contracts`
//! re-indents them, so the leading whitespace here is presentation only.
//! Composition rejects any path, operation or schema key another face also
//! claims — see `console-contracts`.

use console_contracts::{Fragment, NamedYaml, Operation, PathItem};

/// The todos face's contribution to the composed OpenAPI document.
pub const OPENAPI_FRAGMENT: Fragment = Fragment {
    source: "console-todos-rest",
    paths: TODO_PATHS,
    schemas: TODO_SCHEMAS,
    // Shared components owned by the document, not by this face. Everything
    // else this face refs must appear in TODO_SCHEMAS.
    external_schemas: &["ErrorBody", "Timestamp", "Uuid"],
};

const TODO_PATHS: &[PathItem] = &[
    PathItem {
        path: "/api/v1/me/todos",
        operations: &[
            Operation {
                method: "get",
                body: r#"      tags:
        - me
      operationId: listMyTodos
      summary: List the authenticated user's todos, open first then newest first
      security:
      - bearerAuth: []
      parameters:
      - name: include_done
        in: query
        required: false
        description: When true, also return completed todos (default false).
        schema:
          type: boolean
      - name: limit
        in: query
        required: false
        description: Page size (clamped server-side to 1..=200; default 100).
        schema:
          type: integer
          format: int64
      responses:
        '200':
          description: The caller's todos.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TodoPage'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '503':
          description: JWT verification is not configured.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorBody'
"#,
            },
            Operation {
                method: "post",
                body: r#"      tags:
        - me
      operationId: createMyTodo
      summary: Create a todo owned by the authenticated user
      description: >-
        The owner is always the authenticated principal. Scope chips
        (person/team/site/entity refs) and object links (kind+id pairs) are
        validated ref lists of at most 20 entries each. Audited as todo.create.
      security:
      - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateTodoRequest'
      responses:
        '201':
          description: The created todo.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TodoSummary'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '422':
          $ref: '#/components/responses/ValidationError'
        '503':
          description: JWT verification is not configured.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorBody'
"#,
            },
        ],
    },
    PathItem {
        path: "/api/v1/me/todos/{todoId}/done",
        operations: &[Operation {
            method: "post",
            body: r#"      tags:
        - me
      operationId: setMyTodoDone
      summary: Mark one of the authenticated user's todos done or undone
      description: >-
        Explicit target state so the same endpoint supports done AND undo.
        A cross-user id is a 404, never another user's row. Audited as
        todo.done / todo.undone.
      security:
      - bearerAuth: []
      parameters:
      - name: todoId
        in: path
        required: true
        schema:
          $ref: '#/components/schemas/Uuid'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/SetTodoDoneRequest'
      responses:
        '200':
          description: The updated todo.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TodoSummary'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'
        '503':
          description: JWT verification is not configured.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorBody'
"#,
        }],
    },
    PathItem {
        path: "/api/v1/me/todos/{todoId}",
        operations: &[Operation {
            method: "delete",
            body: r#"      tags:
        - me
      operationId: deleteMyTodo
      summary: Delete one of the authenticated user's todos
      description: >-
        A cross-user id is a 404, never another user's row. Audited as
        todo.delete.
      security:
      - bearerAuth: []
      parameters:
      - name: todoId
        in: path
        required: true
        schema:
          $ref: '#/components/schemas/Uuid'
      responses:
        '204':
          description: The todo was deleted.
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'
        '503':
          description: JWT verification is not configured.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorBody'
"#,
        }],
    },
];

const TODO_SCHEMAS: &[NamedYaml] = &[
    NamedYaml {
        name: "TodoRef",
        body: r#"      type: object
      description: >-
        One scope chip or object link: a reference to a domain object by
        kind + id with an optional display-label snapshot. `kind` is an
        extensible free-form string (frontend object-registry kinds), not an
        enum.
      required:
      - kind
      - id
      properties:
        kind:
          type: string
        id:
          type: string
        label:
          type: string
"#,
    },
    NamedYaml {
        name: "TodoSummary",
        body: r#"      type: object
      required:
      - id
      - owner_user_id
      - text
      - scopes
      - links
      - done
      - created_at
      - updated_at
      - done_at
      properties:
        id:
          $ref: '#/components/schemas/Uuid'
        owner_user_id:
          $ref: '#/components/schemas/Uuid'
        text:
          type: string
        scopes:
          type: array
          items:
            $ref: '#/components/schemas/TodoRef'
        links:
          type: array
          items:
            $ref: '#/components/schemas/TodoRef'
        done:
          type: boolean
        created_at:
          $ref: '#/components/schemas/Timestamp'
        updated_at:
          $ref: '#/components/schemas/Timestamp'
        done_at:
          type:
          - string
          - 'null'
          format: date-time
          description: When the todo was first marked done; null while open.
"#,
    },
    NamedYaml {
        name: "TodoPage",
        body: r#"      type: object
      required:
      - items
      properties:
        items:
          type: array
          items:
            $ref: '#/components/schemas/TodoSummary'
"#,
    },
    NamedYaml {
        name: "CreateTodoRequest",
        body: r#"      type: object
      required:
      - text
      properties:
        text:
          type: string
          description: 1..=500 characters after trimming.
        scopes:
          type: array
          description: Scope chips (person/team/site/entity refs); at most 20.
          items:
            $ref: '#/components/schemas/TodoRef'
        links:
          type: array
          description: Object links (kind+id pairs); at most 20.
          items:
            $ref: '#/components/schemas/TodoRef'
"#,
    },
    NamedYaml {
        name: "SetTodoDoneRequest",
        body: r#"      type: object
      required:
      - done
      properties:
        done:
          type: boolean
          description: Explicit target state (true = done, false = undo).
"#,
    },
];
