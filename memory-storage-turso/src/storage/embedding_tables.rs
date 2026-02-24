use crate::TursoStorage;

impl TursoStorage {
    #[allow(dead_code)]
    pub fn get_embedding_table_for_dimension(&self, dimension: usize) -> &'static str {
        match dimension {
            384 => "embeddings_384",
            1024 => "embeddings_1024",
            1536 => "embeddings_1536",
            3072 => "embeddings_3072",
            _ => "embeddings_other",
        }
    }

    #[allow(dead_code)]
    pub fn get_vector_index_for_dimension(&self, dimension: usize) -> Option<&'static str> {
        match dimension {
            384 => Some("idx_embeddings_384_vector"),
            1024 => Some("idx_embeddings_1024_vector"),
            1536 => Some("idx_embeddings_1536_vector"),
            3072 => Some("idx_embeddings_3072_vector"),
            _ => None,
        }
    }
}
