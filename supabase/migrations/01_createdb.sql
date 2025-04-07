-- Enable the pgvector extension for embeddings
CREATE EXTENSION IF NOT EXISTS vector;

-- Create files table
CREATE TABLE files (
    id TEXT PRIMARY KEY, -- file path as unique identifier
    name TEXT NOT NULL,
    content TEXT,
    embedding vector(384), -- OpenAI's default embedding dimension
    inserted_at TIMESTAMP WITH TIME ZONE DEFAULT TIMEZONE('utc'::text, NOW()) NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT TIMEZONE('utc'::text, NOW()) NOT NULL
);

-- Create a GiST index for faster similarity searches on embeddings
CREATE INDEX IF NOT EXISTS files_embedding_idx ON files USING ivfflat (embedding vector_cosine_ops);

-- Add trigger to automatically update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = TIMEZONE('utc'::text, NOW());
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_files_updated_at
    BEFORE UPDATE ON files
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL, -- groups related messages together
    content TEXT NOT NULL,
    "from" TEXT NOT NULL CHECK ("from" IN ('ai', 'user')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT TIMEZONE('utc'::text, NOW()) NOT NULL
);

CREATE FUNCTION query_files (
    file_directory TEXT,
    query_embedding VECTOR(384)
) RETURNS TABLE (
    id TEXT,
    name TEXT,
    content TEXT,
    similarity FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        files.id,
        files.name,
        files.content,
        1 - (files.embedding <=> query_embedding) as similarity
    FROM files
    WHERE starts_with(files.id, file_directory)
    ORDER BY files.embedding <=> query_embedding;
END;
$$ LANGUAGE plpgsql;

-- Create index for faster conversation history retrieval
CREATE INDEX conversations_conversation_id_idx ON conversations(conversation_id);
CREATE INDEX conversations_created_at_idx ON conversations(created_at);

-- Add helpful comments
COMMENT ON TABLE files IS 'Stores file contents and their embeddings for semantic search';
COMMENT ON TABLE conversations IS 'Stores conversation history between users and AI';
COMMENT ON COLUMN files.embedding IS 'Vector embedding of file content for semantic similarity search';
COMMENT ON COLUMN conversations.conversation_id IS 'Groups messages belonging to the same conversation thread';
