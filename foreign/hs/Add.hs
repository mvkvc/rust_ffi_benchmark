{-# LANGUAGE BangPatterns #-}
{-# LANGUAGE ForeignFunctionInterface #-}

module Add where

import Data.Text.Unsafe (inlinePerformIO)
import Foreign.C.Types
import Foreign.Marshal.Unsafe
import Foreign.Ptr
import Foreign.Storable

add :: CInt -> CInt -> CInt
add (CInt x) (CInt y) = CInt (x + y)
{-# INLINE add #-}

data Numbers = Numbers {a :: !CInt, b :: !CInt}

instance Storable Numbers where
  sizeOf _ = 8
  alignment _ = 4
  peek ptr = do
    !a' <- peekByteOff ptr 0
    !b' <- peekByteOff ptr 4
    return $! Numbers a' b'
  poke ptr (Numbers a' b') = do
    pokeByteOff ptr 0 a'
    pokeByteOff ptr 4 b'

add_struct :: Ptr Numbers -> IO CInt
add_struct ptr = do
  let !a' = inlinePerformIO $ peekByteOff ptr 0 :: CInt
      !b' = inlinePerformIO $ peekByteOff ptr 4 :: CInt
  return $! a' + b'
{-# INLINE add_struct #-}

foreign export ccall "add" add :: CInt -> CInt -> CInt
foreign export ccall "add_struct" add_struct :: Ptr Numbers -> IO CInt